FROM node:14.15.3

COPY package.json .
COPY yarn.lock .
RUN yarn

COPY src src

CMD ["yarn", "dev"]