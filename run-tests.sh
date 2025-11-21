#!/bin/bash

# SunBay SoftPOS Backend Test Runner
# This script sets up the test environment and runs tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== SunBay SoftPOS Backend Test Runner ===${NC}\n"

# Check if PostgreSQL is running
echo -e "${YELLOW}Checking PostgreSQL...${NC}"
if ! pg_isready > /dev/null 2>&1; then
    echo -e "${RED}Error: PostgreSQL is not running${NC}"
    exit 1
fi
echo -e "${GREEN}✓ PostgreSQL is running${NC}\n"

# Set test database URL
export TEST_DATABASE_URL=${TEST_DATABASE_URL:-"postgres://postgres:postgres@localhost/softpos_test"}

# Check if test database exists
echo -e "${YELLOW}Checking test database...${NC}"
if ! psql -lqt | cut -d \| -f 1 | grep -qw softpos_test; then
    echo -e "${YELLOW}Creating test database...${NC}"
    createdb softpos_test
    echo -e "${GREEN}✓ Test database created${NC}"
else
    echo -e "${GREEN}✓ Test database exists${NC}"
fi

# Run migrations
echo -e "\n${YELLOW}Running migrations...${NC}"
DATABASE_URL=$TEST_DATABASE_URL sqlx migrate run
echo -e "${GREEN}✓ Migrations complete${NC}\n"

# Parse command line arguments
TEST_TYPE=${1:-"all"}
VERBOSE=${2:-""}

case $TEST_TYPE in
    "unit")
        echo -e "${GREEN}Running unit tests...${NC}\n"
        cargo test --lib $VERBOSE
        ;;
    "integration")
        echo -e "${GREEN}Running integration tests...${NC}\n"
        cargo test --test '*' $VERBOSE
        ;;
    "coverage")
        echo -e "${GREEN}Generating test coverage...${NC}\n"
        if ! command -v cargo-tarpaulin &> /dev/null; then
            echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
            cargo install cargo-tarpaulin
        fi
        cargo tarpaulin --out Html --output-dir coverage
        echo -e "${GREEN}✓ Coverage report generated in coverage/index.html${NC}"
        ;;
    "watch")
        echo -e "${GREEN}Running tests in watch mode...${NC}\n"
        if ! command -v cargo-watch &> /dev/null; then
            echo -e "${YELLOW}Installing cargo-watch...${NC}"
            cargo install cargo-watch
        fi
        cargo watch -x test
        ;;
    "clean")
        echo -e "${YELLOW}Cleaning test database...${NC}"
        dropdb softpos_test
        createdb softpos_test
        DATABASE_URL=$TEST_DATABASE_URL sqlx migrate run
        echo -e "${GREEN}✓ Test database cleaned${NC}"
        ;;
    "all"|*)
        echo -e "${GREEN}Running all tests...${NC}\n"
        cargo test $VERBOSE
        ;;
esac

# Check test result
if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}=== All tests passed! ===${NC}"
else
    echo -e "\n${RED}=== Some tests failed ===${NC}"
    exit 1
fi
