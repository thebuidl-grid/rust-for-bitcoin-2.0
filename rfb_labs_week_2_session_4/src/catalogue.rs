use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Book { pages: u32 },
    Audiobook { minutes: u32 },
    Ebook { size_kb: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoanStatus {
    Available,
    OnLoan { member_id: u32, day_borrowed: u32 },
    Lost,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Item {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub kind: MediaKind,
    pub status: LoanStatus,
}

/// How long each kind of media may be kept, and what it costs to keep it too
/// long. Loan lengths are in whole days; fees are in whole cents.
pub trait LoanTerms {
    fn loan_days(&self) -> u32;

    fn daily_late_fee_cents(&self) -> u32;

    /// Nothing is owed until the loan period runs out, so the overdue count
    /// saturates at zero rather than wrapping on an on-time return.
    fn late_fee_cents(&self, days_held: u32) -> u32 {
        let days_overdue = days_held.saturating_sub(self.loan_days());
        days_overdue.saturating_mul(self.daily_late_fee_cents())
    }
}

impl Item {
    pub fn new(id: u32, title: String, author: String, kind: MediaKind) -> Self {
        Self {
            id,
            title,
            author,
            kind,
            status: LoanStatus::Available,
        }
    }
}

/// The daily fee for anything that can actually run late, in whole cents.
const DAILY_LATE_FEE_CENTS: u32 = 25;

impl LoanTerms for MediaKind {
    fn loan_days(&self) -> u32 {
        match self {
            Self::Book { .. } => 21,
            Self::Audiobook { .. } => 14,
            Self::Ebook { .. } => 7,
        }
    }

    fn daily_late_fee_cents(&self) -> u32 {
        match self {
            // An ebook's licence simply expires, so it is never late.
            Self::Ebook { .. } => 0,
            Self::Book { .. } | Self::Audiobook { .. } => DAILY_LATE_FEE_CENTS,
        }
    }
}

impl LoanTerms for Item {
    fn loan_days(&self) -> u32 {
        self.kind.loan_days()
    }

    fn daily_late_fee_cents(&self) -> u32 {
        self.kind.daily_late_fee_cents()
    }
}

impl fmt::Display for MediaKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Book { pages } => write!(formatter, "book, {pages} pages"),
            Self::Audiobook { minutes } => write!(formatter, "audiobook, {minutes} minutes"),
            Self::Ebook { size_kb } => write!(formatter, "ebook, {size_kb} kB"),
        }
    }
}

impl fmt::Display for LoanStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(formatter, "available"),
            Self::OnLoan {
                member_id,
                day_borrowed,
            } => write!(
                formatter,
                "on loan to member {member_id} since day {day_borrowed}"
            ),
            Self::Lost => write!(formatter, "lost"),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{} \"{}\" by {} ({}) — {}",
            self.id, self.title, self.author, self.kind, self.status
        )
    }
}
