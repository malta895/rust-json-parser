#!/bin/bash

if [ -z "$JSON_PARSER_EXECUTABLE" ]; then
    echo "Error: JSON_PARSER_EXECUTABLE environment variable is not set."
    exit 1
fi

test_json_file() {
    local file_path="$1"
    local expected_exit_code="$2"
    
    echo "testing file $file_path"
    
    cat "$file_path" | $JSON_PARSER_EXECUTABLE
    actual_exit_code=$?
    
    if [ $actual_exit_code -ne $expected_exit_code ]; then
        echo "Test failed for file: $file_path. Expected exit code: $expected_exit_code, Got: $actual_exit_code"
        exit 1
    fi
}

for step_dir in ./tests/step*; do
    
    for invalid_file in $step_dir/invalid*.json; do
        test_json_file "$invalid_file" 1
    done
    
    for valid_file in $step_dir/valid*.json; do
        test_json_file "$valid_file" 0
    done
    
done

echo "All tests passed!"
