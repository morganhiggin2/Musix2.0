use std::env;

pub struct EnvironmentVariables {
    database_s3_path: String,
}

pub fn get_environment_variables() -> Result<EnvironmentVariables, String> {
    // Get the s3 path for the database
    let database_s3_path = match env::var("DATABASE_S3_PATH") {
        Ok(some) => some,
        Err(e) => format!(
            "Could not fetch DATABASE_S3_PATH environment variable: {}",
            e.to_string()
        ),
    };

    let environment_variables = EnvironmentVariables {
        database_s3_path: database_s3_path,
    };

    return Ok(environment_variables);
}
