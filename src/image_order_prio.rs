#[derive(Debug)]

pub enum Priority {
    BestFit,
    Downsize { fact: f32 },
    Upsize { fact: f32 },
    Any
}

impl Priority {
    fn get_rank(&self) -> u32{
	match self {
	    Priority::BestFit => 1,
	    Priority::Downsize { .. } => 2,
	    Priority::Upsize { .. } => 3,
	    Priority::Any => 4,
	}
    }
}

impl PartialEq for Priority {
    fn eq(&self, other: &Self) -> bool {
	match self.cmp(other) {
	    std::cmp::Ordering::Equal => true,
	    _ => false,
	}
    }
}

impl Eq for Priority {}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
	match self {
	    Priority::BestFit => {
		match other {
		    Priority::BestFit => std::cmp::Ordering::Equal,
		    _ => std::cmp::Ordering::Greater,
		}
	    },
	    Priority::Downsize { fact: fact_self } => {
		match other {
		    Priority::BestFit => std::cmp::Ordering::Less,
		    Priority::Downsize { fact: fact_other } => fact_self.total_cmp(fact_other),
		    _ => std::cmp::Ordering::Greater,
		}
	    },
	    Priority::Upsize { fact: fact_self } => {
		match other {
		    Priority::Upsize { fact: fact_other } => fact_other.total_cmp(fact_self),
		    Priority::Any => std::cmp::Ordering::Greater,
		    _ => std::cmp::Ordering::Less,
		}
	    },
	    Priority::Any => {
		match other {
		    Priority::BestFit => std::cmp::Ordering::Less,
		    Priority::Downsize { fact: _ } => std::cmp::Ordering::Less,
		    Priority::Upsize { fact: _ } => std::cmp::Ordering::Less,
		    Priority::Any => std::cmp::Ordering::Equal,
		}
	    },
	}
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
