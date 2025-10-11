sudo mkdir /run/postgresql
sudo chown postgres /run/postgresql
sudo -u postgres pg_ctl -D /var/lib/postgres/data -l logfile start
