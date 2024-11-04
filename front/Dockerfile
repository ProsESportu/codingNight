FROM oven/bun:1-alpine
LABEL authors="tomasz"
COPY package.json .
COPY bun.lockb .
RUN bun install --frozenlockfile
COPY . .
RUN bun run build
ENTRYPOINT ["bun", "run","--bun", "preview", "--host"]