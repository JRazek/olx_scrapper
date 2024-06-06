create table listings (
	id SERIAL PRIMARY KEY,
	url varchar(255) unique not null,
	category int not null,
	query varchar(255),
	title varchar(255) not null,
	price int not null,
	negotiable boolean default false,
	description text,
	location varchar(255),
	date_posted timestamp not null,
	date_updated timestamp,
	first_seen timestamp default current_timestamp,
	last_seen timestamp default current_timestamp
);
