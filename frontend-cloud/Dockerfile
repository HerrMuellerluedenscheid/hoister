# Build stage — use Bun to mirror the dev setup (bun.lock).
FROM oven/bun:1-alpine AS builder

WORKDIR /app

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY . .
RUN bun run build

# Production stage — Node runtime is enough to serve adapter-node output.
FROM node:24-alpine

LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Hoister hosted dashboard (frontend-cloud)"
LABEL io.hoister.container="frontend-cloud"

WORKDIR /app

# Bring across only what the runtime needs: the adapter-node bundle.
# Everything (deps + server) is statically bundled by Vite into build/.
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json

EXPOSE 3000

ENV NODE_ENV=production
ENV PORT=3000
# The dashboard sits behind Caddy on hoister.io. ORIGIN must match the
# externally-visible URL so SvelteKit accepts form-action POSTs.
ENV ORIGIN=https://hoister.io

CMD ["node", "build/index.js"]
