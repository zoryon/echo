#!/bin/bash

# Start the production container
docker compose up -d prod

# Show logs (optional)
docker compose logs -f prod
