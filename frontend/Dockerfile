# Build stage
FROM node:24-slim AS builder

WORKDIR /app

# Copy package files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# Copy source files
COPY . .

# Build the app
RUN NODE_OPTIONS="--max-old-space-size=4096" npm run build

# Production stage
FROM node:24-alpine

LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Deploy and roll back docker images (frontend)"
LABEL io.hoister.container="frontend"

WORKDIR /app

# Copy package files and built output
COPY package.json package-lock.json ./

# Install production dependencies only
RUN npm ci --omit=dev

# Copy built app from builder
COPY --from=builder /app/build ./build
COPY --from=builder /app/.svelte-kit ./.svelte-kit

# Expose port
EXPOSE 3000

# Set environment variables
ENV NODE_ENV=production
ENV PORT=3000
ENV ORIGIN=http://localhost:3000

# Start the app
CMD ["node", "build/index.js"]
