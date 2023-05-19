[![codecov](https://codecov.io/gh/jeremy159/datamize-server/branch/main/graph/badge.svg?token=NZ84G7KHIM)](https://codecov.io/gh/jeremy159/datamize-server)
![CI Build](https://github.com/jeremy159/datamize-server/actions/workflows/main.yml/badge.svg)
![CI Tests](https://github.com/jeremy159/datamize-server/actions/workflows/tests.yml/badge.svg)

# Datamize

A server that gets data from a budget app (in this case YNAB) and exposes some useful data formating through its API.

## Setup

There is currently one secret to setup, with docker secrets.
But first, you need to make sure docker is running in swarm mode. \
You can check if it's running by entering `docker info`. If it is inactive
you can initialize it with `docker swarm init`.

Next you need to create a text file (that should NOT be commited) and paste inside your YNAB Personnal Access Token (PAT).

Once you have that token in a file, run `docker secret create ynab_pat ynab_pat.txt`.

And that's it, you should be able to run the server with access to YNAB's API.

## To get inspiration from

- https://advanced-tools-for-ynab.web.app
- https://beyondrule4.jmmorrissey.com/home
