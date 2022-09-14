//! Contains the [`Acct`] struct representing an account.

#[derive(Debug, Default, PartialEq)]
pub struct Acct {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Acct {
    pub fn deposit(&mut self, amt: f64) -> Result<(), &'static str> {
        if amt > 0.0 {
            self.total += amt;
            self.available += amt;
            Ok(())
        } else {
            Err("cannot deposit negative funds")
        }
    }

    pub fn withdrawal(&mut self, amt: f64) -> Result<(), &'static str> { // TODO: use a better error type
        if self.available < amt {
            return Err("funds not available for withdrawal");
        }
        if amt > 0.0 {
            self.total -= amt;
            self.available -= amt;
            Ok(())
        } else {
            Err("cannot withdrawal negative funds")
        }
    }

    pub fn dispute(&mut self, amt: f64) {
        self.available -= amt;
        self.held += amt;
    }

    pub fn resolve(&mut self, amt: f64) {
        self.available += amt;
        self.held -= amt;
    }

    pub fn chargeback(&mut self, amt: f64) {
        self.held -= amt;
        self.total -= amt;
        self.locked = true;
    }
}

//------------------------------------------------------------------------------
//------------------------------------------------------------------------------
// TESTS
//------------------------------------------------------------------------------
//------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deposit() {
        let mut acct = Acct::default();

        assert!(acct.deposit(1.0).is_ok());
        assert!(acct.deposit(0.0).is_err());
        assert!(acct.deposit(-1.0).is_err());

        assert_eq!(Acct{ available: 1.0, held: 0.0, total: 1.0, locked: false }, acct);
    }

    #[test]
    fn withdrawal() {
        let mut acct = Acct::default();
        _ = acct.deposit(1.0);

        assert!(acct.withdrawal(0.5).is_ok());
        assert!(acct.withdrawal(1.0).is_err());
        assert!(acct.withdrawal(0.0).is_err());
        assert!(acct.withdrawal(-1.0).is_err());

        assert_eq!(Acct{ available: 0.5, held: 0.0, total: 0.5, locked: false }, acct);
    }

    #[test]
    fn dispute_deposit() {
        let mut acct = Acct::default();
        _ = acct.deposit(1.0);

        acct.dispute(1.0);
        assert_eq!(Acct{ available: 0.0, held: 1.0, total: 1.0, locked: false }, acct);

        acct.resolve(1.0);
        assert_eq!(Acct{ available: 1.0, held: 0.0, total: 1.0, locked: false }, acct);

        acct.dispute(1.0);
        acct.chargeback(1.0);
        assert_eq!(Acct{ available: 0.0, held: 0.0, total: 0.0, locked: true }, acct);
    }

    #[test]
    fn dispute_withdraw() {
        let mut acct = Acct::default();
        _ = acct.deposit(1.0);
        _ = acct.withdrawal(0.5);

        acct.dispute(-0.5);
        assert_eq!(Acct{ available: 1.0, held: -0.5, total: 0.5, locked: false }, acct);

        acct.resolve(-0.5);
        assert_eq!(Acct{ available: 0.5, held: 0.0, total: 0.5, locked: false }, acct);

        acct.dispute(-0.5);
        acct.chargeback(-0.5);
        assert_eq!(Acct{ available: 1.0, held: 0.0, total: 1.0, locked: true }, acct);
    }
}
