FROM oven/bun:alpine

WORKDIR /pf-client

COPY . .
RUN bun install

RUN bun run build

USER bun
EXPOSE 3000/tcp

ENTRYPOINT ["bun", "run", "./build/index.js"]
