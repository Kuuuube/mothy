# Mothy

A small Discord bot that i'm using to learn rust. The Code is questionable, I know.

This bot is created with [poise](https://github.com/serenity-rs/poise/) which is a Discord API wrapper.

## Setup

### Postgres

1. Install postgresql

2. Switch user:

    ```
    sudo su - postgres
    ```

3. Init the db:

    ```
    initdb -D /var/lib/postgres/data
    ```

4. Start the database server:

    ```
    pg_ctl -D /var/lib/postgres/data -l logfile start
    ```

    Note: If you get `FATAL:  could not create lock file`, the directory `/run/postgresql` may not be created. Run the following to create and set ownership of it:

    ```
    sudo mkdir /run/postgresql
    sudo chown postgres /run/postgresql
    ```

5. Exit the postgres user with `exit`

### Sqlx

1. Install sqlx-cli

    ```
    cargo install sqlx-cli
    ```

2. Create `.env` and input the following:

    ```
    DATABASE_URL=postgres://postgres@localhost/postgres
    ```

    Note: Replace the `postgres` on the end with whatever your database name is if your are not using the default.

3. Init database:

    ```
    cargo sqlx migrate run
    ```

### Mothy

1. Run `mregister` to register commands.
