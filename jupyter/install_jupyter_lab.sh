#!/bin/bash

# Exit on any error
set -e

# Variables
PYENV_VERSION="3.12.4"
VENV_NAME="jupyterlab"
PYTHON_VERSION="3.12.4"

# Check if pyenv is installed
if ! command -v pyenv &> /dev/null; then
    echo "Error: pyenv is not installed. Please install pyenv first."
    exit 1
fi

# Check if virtual environment already exists
if pyenv virtualenvs | grep -q "$VENV_NAME"; then
    echo "Virtual environment '$VENV_NAME' already exists. Please choose a different name or delete the existing one."
    exit 1
fi

# Create a new virtual environment
echo "Creating virtual environment '$VENV_NAME' with Python $PYTHON_VERSION..."
pyenv virtualenv $PYTHON_VERSION $VENV_NAME

# Activate the virtual environment
echo "Activating virtual environment '$VENV_NAME'..."
eval "$(pyenv init -)"
pyenv activate $VENV_NAME

# Upgrade pip to the latest version
echo "Upgrading pip..."
pip install --upgrade pip

# Install minimal JupyterLab, classic Jupyter Notebook, and required packages
echo "Installing JupyterLab, Jupyter Notebook, and jupyter_contrib_nbextensions..."
pip install jupyterlab notebook


# Verify installation
echo "Verifying JupyterLab installation..."
jupyter lab --version

# Deactivate the virtual environment
echo "Deactivating virtual environment..."
pyenv deactivate

echo "Setup complete! To use JupyterLab, activate the environment with:"
echo "pyenv activate $VENV_NAME"
echo "Then run 'jupyter lab' to start JupyterLab or 'jupyter notebook' for the classic Notebook interface."
