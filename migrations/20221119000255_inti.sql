-- Add migration script here
create table todos (
  id serial ,
  text text not null,
  completed boolean not null default false,
  primary key (`id`)
);