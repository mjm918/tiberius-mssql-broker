use std::fmt::Write;
use sea_query::Iden;

#[derive(Debug)]
pub enum Character {
	Table,
	Id,
	AccountCode,
	Description,
	ChartSequence,
	DRCR,
	OptimisticLockField,
	ChartSequencePH,
	AccountCodePH,
	RowIndex,
}

pub type Char = Character;

impl Iden for Character {
	fn unquoted(&self, s: &mut dyn Write) {
		write!(
			s,
			"{}",
			match self {
				Self::Table => "character",
				Self::Id => "id",
				Self::AccountCode => "AccountCode",
				Self::Description => "Description",
				Self::ChartSequence => "ChartSequence",
				Self::DRCR => "DRCR",
				Self::OptimisticLockField => "OptimisticLockField",
				Self::ChartSequencePH => "ChartSequencePH",
				Self::AccountCodePH => "AccountCodePH",
				Self::RowIndex => "RowIndex",
			}
		)
			.unwrap();
	}
}