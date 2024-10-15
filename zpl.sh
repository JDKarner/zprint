#!/bin/bash

# Function to list available ZTC printers with a base name
list_printers() {
  local base_name="$1"
  lpstat -p | awk -v base="$base_name" '$2 ~ base {print $2}'
}

# Define the base name of the printer
PRINTER_BASE="ZTC-ZP-450-200dpi"

# List available printers with the base name
AVAILABLE_PRINTERS=($(list_printers "$PRINTER_BASE"))

# Check if there are any printers available
if [ ${#AVAILABLE_PRINTERS[@]} -eq 0 ]; then
  echo "No printers found with base name '$PRINTER_BASE'."
  exit 1
fi

# Select the first available printer
SELECTED_PRINTER="${AVAILABLE_PRINTERS[0]}"
PRINT_COMMAND="lpr -P $SELECTED_PRINTER -o raw"

# Function to check if the printer is available
check_printer() {
  lpstat -p | grep -q "$SELECTED_PRINTER"
}

# Check if the selected printer is available
if ! check_printer; then
  echo "The selected printer ($SELECTED_PRINTER) is not currently available."
  echo "Please ensure the printer is plugged in and turned on, then press Enter to retry..."
  read -r
  if ! check_printer; then
    echo "The printer is still not detected. Exiting."
    exit 1
  fi
fi

# Path to the Downloads folder
DOWNLOADS_FOLDER="$HOME/Downloads"

# Find all .zpl files in the Downloads folder
ZPL_FILES=($DOWNLOADS_FOLDER/*.ZPL)

# Check if there are no .zpl files
if [ ${#ZPL_FILES[@]} -eq 0 ]; then
  echo "No .zpl files found in $DOWNLOADS_FOLDER."
  exit 1
fi

# If there is more than one .zpl file, ask for permission to delete them
if [ ${#ZPL_FILES[@]} -gt 1 ]; then
  echo "Multiple .zpl files found. Do you want to delete them all after printing? (y/n)"
  read -r DELETE_CONFIRMATION

  if [[ "$DELETE_CONFIRMATION" != "y" ]]; then
    echo "Files will not be deleted. Exiting."
    exit 0
  fi
fi

# Print each .zpl file and wait for 30 seconds
for ZPL_FILE in "${ZPL_FILES[@]}"; do
  if [ -f "$ZPL_FILE" ]; then
    echo "Printing $ZPL_FILE using $SELECTED_PRINTER..."
    $PRINT_COMMAND "$ZPL_FILE"
    sleep 30

    echo "Deleting $ZPL_FILE..."
    rm "$ZPL_FILE"
  else
    echo "File $ZPL_FILE does not exist."
  fi
done

# If there were multiple files, ask the user to re-download them
if [ ${#ZPL_FILES[@]} -gt 1 ]; then
  echo "Please re-download the .zpl files."
fi

echo "Script completed."
