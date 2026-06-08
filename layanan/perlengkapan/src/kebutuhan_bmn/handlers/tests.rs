use crate::kebutuhan_bmn::*;

#[test]
fn test_pagination_validation() {
    let valid = PaginationQuery {
        page: 1,
        per_page: 20,
    };
    assert!(valid.validate().is_ok());

    let invalid_page = PaginationQuery {
        page: 0,
        per_page: 20,
    };
    assert!(invalid_page.validate().is_err());

    let invalid_per_page = PaginationQuery {
        page: 1,
        per_page: 0,
    };
    assert!(invalid_per_page.validate().is_err());

    let too_large = PaginationQuery {
        page: 1,
        per_page: 2000,
    };
    assert!(too_large.validate().is_err());
}
