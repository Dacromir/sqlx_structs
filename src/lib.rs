pub struct Employee {
    pub id: u64,
    pub name: String,
    pub team: Team,
}

pub struct EmployeeWithId {
    pub id: u64,
    pub name: String,
    pub team: TeamId,
}

pub type TeamId = u64;

pub struct Team {
    pub id: TeamId,
    pub name: String,
}
