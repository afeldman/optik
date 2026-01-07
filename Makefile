.PHONY: help install dev test build clean lint format

help:
	@echo "optik - High-performance camera manager"
	@echo ""
	@echo "Available targets:"
	@echo "  install    Install in production mode"
	@echo "  dev        Install in development mode"
	@echo "  test       Run tests"
	@echo "  build      Build Rust extension"
	@echo "  clean      Clean build artifacts"
	@echo "  lint       Run linters"
	@echo "  format     Format code"

install:
	pip install -e .

dev:
	pip install -e ".[dev]"
	pre-commit install

test:
	pytest -v

build:
	maturin develop

clean:
	rm -rf build dist *.egg-info
	rm -rf target
	rm -rf .pytest_cache .coverage htmlcov
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

lint:
	ruff check src/python tests
	mypy src/python

format:
	black src/python tests
	ruff check --fix src/python tests
