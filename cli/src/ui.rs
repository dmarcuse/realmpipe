use cursive::event::Key;
use cursive::views::{Dialog, TextView};
use cursive::Cursive;

pub fn run() {
    // initialize cursive
    let mut siv = Cursive::default();

    // set up menus
    siv.set_autohide_menu(false);
    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
    siv.menubar().add_leaf("Quit", |s| {
        s.add_layer(
            Dialog::new()
                .title("Really Quit?")
                .content(TextView::new(
                    "Do you really want to quit? All clients will be disconnected.",
                ))
                .button("Cancel", |s| {
                    s.pop_layer();
                })
                .button("Continue", |s| s.quit()),
        )
    });

    siv.run();
}
