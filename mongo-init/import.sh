#!/bin/bash
mongoimport --host localhost --db news --collection articles --type json --file /docker-entrypoint-initdb.d/data.json --jsonArray --authenticationDatabase admin -u admin -p password123
