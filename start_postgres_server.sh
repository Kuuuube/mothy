sudo mkdir /run/postgresql
sudo chown postgres /run/postgresql

# if this command doesn't work switch to the postgres user and run it instead
# ex:
# sudo su - postgres
# pg_ctl -D /var/lib/postgres/data -l logfile start
# exit
sudo -u postgres pg_ctl -D /var/lib/postgres/data -l logfile start
