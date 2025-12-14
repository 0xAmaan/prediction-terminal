#!/bin/bash
# Generate market embeddings for semantic matching
# This should be run once initially and then periodically (e.g., daily)

set -e

echo "🚀 Generating market embeddings..."
echo "This will:"
echo "  - Fetch all active markets (up to 500)"
echo "  - Generate OpenAI embeddings for each market"
echo "  - Store embeddings in data/embeddings.db"
echo "  - Cost: ~\$0.02-0.05 for 500 markets"
echo ""

# Check if OPENAI_API_KEY is set
if [ -z "$OPENAI_API_KEY" ]; then
    echo "❌ Error: OPENAI_API_KEY not set"
    echo "Please add it to your .env.local file"
    exit 1
fi

echo "✅ OpenAI API key found"
echo ""

# For now, we'll add an API endpoint to trigger this
echo "Starting server to generate embeddings..."
echo "Once the server starts, run:"
echo ""
echo "  curl -X POST http://localhost:3001/api/admin/generate-embeddings"
echo ""
echo "Or wait for automatic generation on first run"
