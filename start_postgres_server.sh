sudo mkdir /run/postgresql
sudo chown postgres /run/postgresql

echo
echo If the following command doesn\'t work switch to the postgres user and run it instead
echo ex:
echo \`\`\`
echo $ sudo su - postgres
echo $ pg_ctl -D /var/lib/postgres/data -l logfile start
echo $ exit
echo \`\`\`
echo
sudo -u postgres pg_ctl -D /var/lib/postgres/data -l logfile start
