psql -c "CREATE USER olx_scrapper_root WITH PASSWORD 'pass';"

psql -c "CREATE DATABASE olx_data WITH OWNER olx_scrapper_root;"

psql -c "GRANT ALL PRIVILEGES ON DATABASE olx_data TO olx_scrapper_root;"

psql -U olx_scrapper_root -d olx_data -f db/setup_db.sql
