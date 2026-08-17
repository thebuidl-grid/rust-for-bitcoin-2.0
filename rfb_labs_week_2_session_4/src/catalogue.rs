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

    fn late_fee_cents(&self, days_held: u32) -> u32 {
         // TODO(Part 4): the shared fee formula lives here so neither impl
        // repeats it. A loan returned on time owes nothing.
        let loan_days = self.loan_days();
        if days_held > loan_days {
            (days_held - loan_days) * self.daily_late_fee_cents()
        } else {
            0
        }
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

impl LoanTerms for MediaKind {
    fn loan_days(&self) -> u32 {
         // TODO(Part 4): books 21, audiobooks 14, ebooks 7.
        // todo!("return the loan length")
        match self {
            MediaKind::Book { pages } => 21,
            MediaKind::Audiobook { minutes } => 14,
            MediaKind::Ebook { size_kb } => 7,
        }
    }

    fn daily_late_fee_cents(&self) -> u32 {
         // TODO(Part 4): 25 cents a day, except ebooks, which are never late.
        // todo!("return the daily late fee")
         match self {
            MediaKind::Ebook { .. } => 0,
            MediaKind::Book{..}| MediaKind::Audiobook{..} => 25,

        // match self {
        //     MediaKind::Ebook { .. } => 0,
        //     _ => 25,
        }
    }
}

impl LoanTerms for Item {
    fn loan_days(&self) -> u32 {
         // TODO(Part 4): an item's terms come from its kind.
        // todo!("return the loan length")
        self.kind.loan_days()
    }

    fn daily_late_fee_cents(&self) -> u32 {
        // TODO(Part 4)
        // todo!("return the daily late fee")
        self.kind.daily_late_fee_cents()
    }
}

impl fmt::Display for MediaKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 4): mention the data the variant carries.
        // todo!("display a media kind")
        match self {
            MediaKind::Book { pages } => write!(formatter, "Book ({} pages)", pages),
            MediaKind::Audiobook { minutes } => write!(formatter, "Audiobook ({} min)", minutes),
            MediaKind::Ebook { size_kb } => write!(formatter, "Ebook ({} KB)", size_kb),
        }
    }
}

impl fmt::Display for LoanStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
         // TODO(Part 4): an on-loan item should name its borrower.
        // todo!("display a loan status")
        match self {
            LoanStatus::Available => write!(formatter, "Available"),
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => {
                write!(
                    formatter,
                    "On loan to member {} (day {})",
                    member_id, day_borrowed
                )
            }
            LoanStatus::Lost => write!(formatter, "Lost"),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 4)
        // todo!("display an item")
        write!(
            formatter,
            "\"{}\" by {} [{}] ({})",
            self.title, self.author, self.id, self.status
        )
    }
}
