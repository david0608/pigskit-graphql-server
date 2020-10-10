pub enum Clause {
    Empty,
    Entries(String),
    #[allow(dead_code)]
    Parentheses(String),
}

impl Clause {
    pub fn new() -> Self {
        Clause::Empty
    }

    pub fn equal<L: std::fmt::Display, R: std::fmt::Display>(left: L, right: R) -> Self {
        Clause::Entries(format!("{} = {}", left, right))
    }

    pub fn like<L: std::fmt::Display, R: std::fmt::Display>(left: L, right: R) -> Self {
        Clause::Entries(format!("{} LIKE {}", left, right))
    }

    pub fn inner(&self) -> &String {
        match self {
            Clause::Empty => panic!(),
            Clause::Entries(inner) | Clause::Parentheses(inner) => inner,
        }
    }

    #[allow(dead_code)]
    pub fn parentheses(&mut self) {
        match self {
            Clause::Entries(inner) => {
                *self = Clause::Parentheses(format!("({})", inner));
            }
            Clause::Empty | Clause::Parentheses(_) => {}
        }
    }

    pub fn and(&mut self, clause: Clause) {
        match self {
            Clause::Empty => {
                *self = clause
            }
            Clause::Entries(inner) | Clause::Parentheses(inner) => {
                *self = Clause::Entries(format!("{} AND {}", inner, clause.inner()))
            }
        }
    }
    
    #[allow(dead_code)]
    pub fn or(&mut self, clause: Clause) {
        match self {
            Clause::Empty => {
                *self = clause
            }
            Clause::Entries(inner) | Clause::Parentheses(inner) => {
                *self = Clause::Entries(format!("{} OR {}", inner, clause.inner()))
            }
        }
    }
}

impl std::fmt::Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Clause::Empty => {
                write!(f, "")
            }
            Clause::Entries(inner) | Clause::Parentheses(inner) => {
                write!(f, " WHERE {}", inner)
            }
        }
    }
}