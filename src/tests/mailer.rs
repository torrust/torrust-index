use lettre::Message;

use crate::mailer::{build_content, build_letter};

#[test]
fn it_should_build_a_letter() {
    let builder = Message::builder()
        .from("from@a.b.c".parse().unwrap())
        .reply_to("reply@a.b.c".parse().unwrap())
        .to("to@a.b.c".parse().unwrap());

    let _letter = build_letter("https://a.b.c/", "user", builder).unwrap();
}

#[test]
fn it_should_build_content() {
    let (plain_body, html_body) = build_content("https://a.b.c/", "user").unwrap();
    assert_ne!(plain_body, "");
    assert_ne!(html_body, "");
}
