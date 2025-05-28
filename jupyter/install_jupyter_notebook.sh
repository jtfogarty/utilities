#!/bin/bash

# Exit on any error
set -e

# Variables
PYENV_VERSION="3.12.4"
VENV_NAME="jupyter_notebook"
PYTHON_VERSION="3.12.4"
NOTEBOOK_VERSION="6.5.4"

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

# Install specific version of Jupyter Notebook and jupyter_contrib_nbextensions
echo "Installing Jupyter Notebook (version $NOTEBOOK_VERSION) and jupyter_contrib_nbextensions..."
pip install notebook==$NOTEBOOK_VERSION jupyter_contrib_nbextensions

# Install the nbextensions
echo "Installing Jupyter contrib nbextensions..."
jupyter contrib nbextension install --user

# Enable the init_cell/main nbextension
echo "Enabling init_cell/main nbextension..."
jupyter nbextension enable init_cell/main

# Verify installation
echo "Verifying Jupyter Notebook installation..."
jupyter notebook --version

# Deactivate the virtual environment
echo "Deactivating virtual environment..."
pyenv deactivate

echo "Setup complete! To use Jupyter Notebook, activate the environment with:"
echo "pyenv activate $VENV_NAME"
echo "Then run 'jupyter notebook' to start the classic Notebook interface."
