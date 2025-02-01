pub struct EnvironmentVariables {
    //database_s3_uri: String,
}

pub fn get_environment_variables() -> Result<EnvironmentVariables, String> {
    // Get the s3 path for the database
    /*let database_s3_uri = match env::var("DATABASE_S3_URI") {
        Ok(some) => some,
        Err(e) => format!(
            "Could not fetch DATABASE_S3_PATH environment variable: {}",
            e.to_string()
        ),
    };*/

    let environment_variables = EnvironmentVariables {
        //database_s3_uri: database_s3_uri,
    };

    return Ok(environment_variables);
}

impl EnvironmentVariables {
    /*pub fn get_database_s3_uri(&self) -> &String {
        &self.database_s3_uri
    }*/
}
