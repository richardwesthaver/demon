use std::net::SocketAddr;

pub use prometheus::{
  self,
  core::{
    AtomicF64 as F64, AtomicI64 as I64, AtomicU64 as U64, GenericCounter as Counter,
    GenericCounterVec as CounterVec, GenericGauge as Gauge, GenericGaugeVec as GaugeVec,
  },
  exponential_buckets, Error as PrometheusError, Histogram, HistogramOpts, HistogramVec, Opts,
  Registry,
};
use prometheus::{core::Collector, Encoder, TextEncoder};

mod sourced;

pub use sourced::{MetricSource, SourcedCounter, SourcedGauge, SourcedMetric};

pub fn register<T: Clone + Collector + 'static>(
  metric: T,
  registry: &Registry,
) -> Result<T, PrometheusError> {
  registry.register(Box::new(metric.clone()))?;
  Ok(metric)
}

mod known_os {
  use hyper::{http::StatusCode, Body, Request, Response};

  use super::*;

  #[derive(Debug, derive_more::Display, derive_more::From)]
  pub enum Error {
    /// Hyper internal error.
    Hyper(hyper::Error),
    /// Http request error.
    Http(hyper::http::Error),
    /// i/o error.
    Io(std::io::Error),
    #[display(fmt = "Prometheus port {} already in use.", _0)]
    PortInUse(SocketAddr),
  }

  impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
      match self {
        Error::Hyper(error) => Some(error),
        Error::Http(error) => Some(error),
        Error::Io(error) => Some(error),
        Error::PortInUse(_) => None,
      }
    }
  }

  async fn request_metrics(
    req: Request<Body>,
    registry: Registry,
  ) -> Result<Response<Body>, Error> {
    if req.uri().path() == "/metrics" {
      let metric_families = registry.gather();
      let mut buffer = vec![];
      let encoder = TextEncoder::new();
      encoder.encode(&metric_families, &mut buffer).unwrap();

      Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", encoder.format_type())
        .body(Body::from(buffer))
        .map_err(Error::Http)
    } else {
      Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not found."))
        .map_err(Error::Http)
    }
  }
}
