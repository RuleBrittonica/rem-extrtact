import os
import re
import csv

# Input test file containing the unit tests
input_file = '0_extract_tests.rs'  # Replace with your actual input file

# Output directories for input and output .rs files
input_dir = 'input'
output_dir = 'correct_output'
os.makedirs(input_dir, exist_ok=True)
os.makedirs(output_dir, exist_ok=True)

# CSV file to store the cursor positions
csv_file = '0_test_info.csv'

# Regex patterns for extracting the test name and raw strings
test_name_pattern = re.compile(r'fn (\w+)\(')
raw_string_pattern = re.compile(r'r#"(.*?)"#', re.DOTALL)

# Function to find cursor positions ($0) in the code
def find_cursor_position(code):
    cursor_index = code.find('$0')
    return cursor_index

# Returns the distance between the end of the first cursor and the start of the
# second cursor
# E.g., for the code:
# fn foo() {
#     foo($01 + 1$0);
# }
# We would return 5 (len(" + 1") = 5)
def distance_between_cursors(code):
    cursor_indices = [m.start() for m in re.finditer(r'\$0', code)]
    if len(cursor_indices) < 2:
        return 0
    return cursor_indices[1] - cursor_indices[0] - 2  # Subtract 2 to account for the $0 characters

# Function to process each test and generate files and CSV entries
def process_tests(input_file, csv_file):
    with open(input_file, 'r') as file, open(csv_file, 'w', newline='') as csvfile:
        csv_writer = csv.writer(csvfile)
        csv_writer.writerow(['Test name', 'Input cursor 1 location', 'Input cursor 2 location'])

        content = file.read()
        test_blocks = content.split('#[test]')

        for test_block in test_blocks[1:]:  # Skip the first element since it's before the first test

            test_name_match = test_name_pattern.search(test_block)
            raw_strings = raw_string_pattern.findall(test_block)


            if test_name_match and len(raw_strings) == 2:
                test_name = test_name_match.group(1)
                input_code = raw_strings[0].strip()  # Remove leading and trailing whitespace
                output_code = raw_strings[1].strip()  # Remove leading and trailing whitespace

                # Find cursor positions in the input code
                input_cursor_1 = find_cursor_position(input_code)
                distance_between_cursors_ = distance_between_cursors(input_code)
                input_cursor_2 = input_cursor_1 + distance_between_cursors_

                print(f"Processing test: {test_name}, Input cursor 1: {input_cursor_1}, Input cursor 2: {input_cursor_2}")


                input_code_cleaned = input_code.replace('$0', '')
                output_code_cleaned = output_code.replace('$0', '')

                with open(os.path.join(input_dir, f'{test_name}.rs'), 'w') as infile:
                    infile.write(input_code_cleaned)
                with open(os.path.join(output_dir, f'{test_name}.rs'), 'w') as outfile:
                    outfile.write(output_code_cleaned)

                # Write character-based cursor information to CSV
                csv_writer.writerow([test_name, input_cursor_1, input_cursor_2])


# Run the script
process_tests(input_file, csv_file)
print(f"Processing complete. Files are saved in '{input_dir}' and '{output_dir}', and the CSV file is '{csv_file}'.")

# Open up the CSV file, and sort the rows alphabetically by the test name
import pandas as pd
df = pd.read_csv(csv_file)
df = df.sort_values(by='Test name')
df.to_csv(csv_file, index=False)
print(f"CSV file sorted alphabetically by test name.")

# Go through every file in input and expected_output, and remove the first line,
# if it begins with // (since it's a comment)

affected_files = []

for directory in [input_dir, output_dir]:
    for filename in os.listdir(directory):
        with open(os.path.join(directory, filename), 'r') as file:
            lines = file.readlines()
            if lines[0].startswith('//'):
                affected_files.append(os.path.join(directory, filename))
                with open(os.path.join(directory, filename), 'w') as file:
                    file.writelines(lines[1:])
print("Removed comments from the first line of each file in the input and output directories.")
for file in affected_files:
    print(f"\t- {file}")
