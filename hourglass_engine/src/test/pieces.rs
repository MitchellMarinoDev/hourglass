use crate::CastleRights;

#[test]
fn revoke_castle_rights() {
    let mut castle_rights = CastleRights::all();
    castle_rights.revoke(CastleRights::WhiteKingSide);
    assert_eq!(
        castle_rights,
        CastleRights::WhiteQueenSide | CastleRights::BlackQueenSide | CastleRights::BlackKingSide
    );
}
