sudo mkdir /run/postgresql
sudo chown postgres /run/postgresql
pg_ctl -D /var/lib/postgres/data -l logfile start
