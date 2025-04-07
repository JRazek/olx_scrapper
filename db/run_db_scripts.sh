#!/bin/bash

# Database credentials
DB_USER="olx_scrapper_root"
DB_USER_PASS="pass"
DB_NAME="olx_data"
DB_HOST="localhost"  # Change if your database host is different

# Directory containing the SQL scripts
SQL_DIR="db/sql"

psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_USER_PASS';"

psql -c "CREATE DATABASE $DB_NAME WITH OWNER $DB_USER;"

psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"


# Check if the directory exists
if [ ! -d "$SQL_DIR" ]; then
  echo "Directory $SQL_DIR does not exist. Exiting."
  exit 1
fi

# Find all SQL scripts in the directory, sort them lexicographically, and iterate over them
for SQL_FILE in $(ls "$SQL_DIR"/*.sql | sort); do
  echo "Executing $SQL_FILE..."
  psql -U "$DB_USER" -d "$DB_NAME" -h "$DB_HOST" -f "$SQL_FILE"
  if [ $? -ne 0 ]; then
    echo "Error executing $SQL_FILE. Exiting."
    exit 1
  fi
done

echo "All scripts executed successfully."

