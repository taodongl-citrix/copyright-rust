FROM ci-local-docker.repo.citrite.net/ubuntu:22.04
RUN apt-get update && apt-get install -y git && apt-get clean
WORKDIR /app
COPY ./build .
ENTRYPOINT /app/run.sh
