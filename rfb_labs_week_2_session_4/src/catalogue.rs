use std::fmt;

#[derive(Debug, PartialOrd, Ord, Clone, Copy, PartialEq, Eq)]
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
        // (Part 4): the shared fee formula lives here so neither impl
        // repeats it. A loan returned on time owes nothing.

        let mut fee_rate = 0;
        if days_held > self.loan_days() {
            fee_rate = (days_held - self.loan_days()) * self.daily_late_fee_cents();
        }
        fee_rate
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
        // (Part 4): books 21, audiobooks 14, ebooks 7.
        //todo!("return the loan length")

        match self {
            MediaKind::Audiobook { .. } => 14,
            MediaKind::Book { .. } => 21,
            MediaKind::Ebook { .. } => 7,
        }
    }

    fn daily_late_fee_cents(&self) -> u32 {
        // (Part 4): 25 cents a day, except ebooks, which are never late.

        match self {
            MediaKind::Audiobook { .. } => 25,
            MediaKind::Book { .. } => 25,
            _ => 0,
        }
    }
}

impl LoanTerms for Item {
    fn loan_days(&self) -> u32 {
        // (Part 4): an item's terms come from its kind.
        self.kind.loan_days()
    }

    fn daily_late_fee_cents(&self) -> u32 {
        // (Part 4)
        self.kind.daily_late_fee_cents()
    }
}

impl fmt::Display for MediaKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // (Part 4): mention the data the variant carries.
        match self {
            MediaKind::Audiobook { minutes } => {
                write!(formatter, "Audiobook has {minutes} minutes")
            }
            MediaKind::Book { pages } => write!(formatter, "The Book has {pages} pages"),
            MediaKind::Ebook { size_kb } => write!(formatter, "The Ebook is of size {size_kb} kb"),
        }
    }
}

impl fmt::Display for LoanStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // (Part 4): an on-loan item should name its borrower.
        match self {
            LoanStatus::Available => write!(formatter, "The item is available"),
            LoanStatus::Lost => write!(formatter, "The item is lost"),
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => write!(
                formatter,
                "The item was borrowed by member with id {member_id} on {day_borrowed}"
            ),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // (Part 4)

        write!(
            formatter,
            "Item Id: {}, Item tttle: {}, Item author: {}, Item kind: {}, Items status: {}",
            self.id, self.title, self.author, self.kind, self.status
        )
    }
}
