use support::project::TestProject;

/// Creates and sets up a clean database with the name bigneon_{name}, by doing the following:
/// * Drops any existing DB with that name
/// * Creates a new Database with name bigneon_{name}
/// * Runs all the migrations on the DB
/// * Sets the ROLE passwords as given in the .env file (or environment)
/// * Creates a set of DB connections for each role
///
/// The [TestProject] struct that is returned as some convenience functions for testing the database contents
pub fn create_database_and_connections(name: &str) -> TestProject {
    let mut project = TestProject::new(name);
    let result = project.command("setup").run();
    assert!(result.is_success(), "Database setup failed {:?}", result);
    //    project.set_role_passwords();
    project.connections.connect();
    assert!(project.db_exists());
    println!("Database {} created", project.database_name);
    project
}
