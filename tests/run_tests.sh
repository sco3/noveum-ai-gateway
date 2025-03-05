#!/bin/bash
# Run the AI Gateway integration tests
# This script helps set up and run the integration tests

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================================${NC}"
echo -e "${BLUE}       AI Gateway Integration Test Runner               ${NC}"
echo -e "${BLUE}=========================================================${NC}"

# Check if .env.test exists
if [ -f ".env.test" ]; then
    ENV_FILE=".env.test"
    echo -e "${GREEN}Found .env.test in project root${NC}"
elif [ -f "tests/.env.test" ]; then
    ENV_FILE="tests/.env.test"
    echo -e "${GREEN}Found .env.test in tests directory${NC}"
else
    echo -e "${YELLOW}No .env.test file found. Creating one from template...${NC}"
    if [ -f "tests/.env.test.example" ]; then
        cp tests/.env.test.example .env.test
        ENV_FILE=".env.test"
        echo -e "${GREEN}Created .env.test from template${NC}"
        echo -e "${YELLOW}Please edit .env.test to add your API keys, then run this script again${NC}"
        exit 0
    else
        echo -e "${RED}Error: tests/.env.test.example not found. Cannot create .env.test${NC}"
        exit 1
    fi
fi

# Check if the user wants to run specific provider tests
if [ "$1" != "" ]; then
    PROVIDER=$1
    echo -e "${BLUE}Running tests for provider: ${PROVIDER}${NC}"
    echo -e "${YELLOW}Make sure the AI Gateway is running with ENABLE_ELASTICSEARCH=true${NC}"
    
    # Run the tests with environment variables from .env.test
    cargo test --test run_integration_tests ${PROVIDER} -- --nocapture
else
    # No provider specified, run all tests
    echo -e "${BLUE}Running all integration tests${NC}"
    echo -e "${YELLOW}Make sure the AI Gateway is running with ENABLE_ELASTICSEARCH=true${NC}"
    
    # Run the tests with environment variables from .env.test
    cargo test --test run_integration_tests -- --nocapture
fi 