#!/bin/bash
FROM node:18
WORKDIR /usr/src/app
ENV PRODUCTION=true
EXPOSE 8080
EXPOSE 80
EXPOSE 443
EXPOSE 8443
COPY package*.json ./
RUN npm install
COPY . .

CMD [ "npm", "run", "start" ]
