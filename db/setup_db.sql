create table categories (
	id serial primary key,
	name varchar(255) unique not null,
	default_query varchar(255)
);

create table listings (
	id serial primary key,
	url varchar(255) unique not null,
	category int not null,
	title varchar(255) not null,
	price int not null,
	negotiable boolean default false,
	description text,
	location varchar(255),
	date_posted timestamp not null,
	date_updated timestamp,
	first_seen timestamp default current_timestamp,
	last_seen timestamp default current_timestamp,

	constraint fk_category
	foreign key(category)
	references categories(id)
	on delete no action
);
