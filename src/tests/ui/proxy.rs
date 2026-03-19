use crate::cache::image::manager::Error;
use crate::ui::proxy::map_error_to_image;

#[test]
fn map_error_url_is_unreachable() {
    let img = map_error_to_image(&Error::UrlIsUnreachable);
    assert_eq!(&img[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn map_error_url_is_not_an_image() {
    let img = map_error_to_image(&Error::UrlIsNotAnImage);
    assert_eq!(&img[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn map_error_image_too_big() {
    let img = map_error_to_image(&Error::ImageTooBig);
    assert_eq!(&img[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn map_error_user_quota_met() {
    let img = map_error_to_image(&Error::UserQuotaMet);
    assert_eq!(&img[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn map_error_unauthenticated() {
    let img = map_error_to_image(&Error::Unauthenticated);
    assert_eq!(&img[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn cached_images_are_consistent() {
    let a = map_error_to_image(&Error::UrlIsUnreachable);
    let b = map_error_to_image(&Error::UrlIsUnreachable);
    assert_eq!(a, b, "LazyLock-cached images should be identical across calls");
}
