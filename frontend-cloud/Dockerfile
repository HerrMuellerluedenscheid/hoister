# Build stage — npm is the single source of truth for dependencies here
# (package-lock.json). Dependabot only maintains the npm lockfile, so keeping
# a second bun.lock alongside it drifted on every bump and broke the build.
FROM node:24-slim AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

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
