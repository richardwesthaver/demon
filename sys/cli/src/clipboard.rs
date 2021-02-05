use clipboard::{ClipboardContext, ClipboardProvider};

fn copy(ui: &mut Cursive) {
    let sel = ui
        .find_name::<SelectView<TreeEntry>>("results")
        .unwrap()
        .selection();

    if sel.is_none() {
        return;
    }
		if let Err(err) = || -> Result<()> {
				let result = sel.unwrap().result()?;
				let mut ctx = clipboard::ClibpardContext::new()?;
				ctx.set_contents(result)?;
				Ok(())
		}
}
