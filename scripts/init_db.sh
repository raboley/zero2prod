#!/usr/bin/env bash
set -x
set -eo pipefail

# check if custom parameter has been set otherwise use default values
DB_PORT="${POSTGRES_PORT:=5432}"
SUPER_USER="${SUPER_USER:=postgres}"
SUPER_USER_PASSWORD="${SUPER_USER_PASSWORD:=password}"

APP_USER="${APP_USER:=app}"
APP_USER_PWD="${APP_USER_PWD:=secret}"
APP_DB_NAME="${APP_DB_NAME:=newsletter}"

# launch postgres using docker
CONTAINER_NAME="postgres"
docker run \
    --env POSTGRES_USER=${SUPER_USER} \
    --env POSTGRES_PASSWORD=${SUPER_USER_PASSWORD} \
    --health-cmd="pg_isready -U ${SUPER_USER} || exit 1" \
    --health-interval=1s \
    --health-timeout=5s \
    --health-retries=5 \
    --publish "${DB_PORT}:5432" \
    --detach \
    --name "${CONTAINER_NAME}" \
    postgres -N 1000
    # ^ Increase the max number of connections for testing purposes

until [ \
    "$(docker inspect -f "{{.State.Health.Status}}" ${CONTAINER_NAME})" == \
    "healthy" \
]; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

CREATE_QUERY="CREATE USER ${APP_USER} WITH PASSWORD '${APP_USER_PWD}';"
docker exec -t "${CONTAINER_NAME}" psql -U "${SUPER_USER}" -c "${CREATE_QUERY}"

GRANT_QUERY="ALTER USER ${APP_USER} CREATEDB;"
docker exec -t "${CONTAINER_NAME}" psql -U "${SUPER_USER}" -c "${GRANT_QUERY}"
