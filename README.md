# Json Revisor
A way to make changes to changing Json data

# Use case
This project was made to modify the DiamondFire action dump to include more useful data for programming languages that compile to DiamondFire templates.

# How to use
## NOTE: Most of this isn't implemented, so these steps will not work

## Getting started
To create a Json Revisor™️ project, you first need to run `json-revisor init` (unimplemented) which will initialize the project by asking you some questions.
(Or just `json-revisor init_default` to get the default file)
Your project file should look something like this.
```json
{
    "name": "",
    "description": "",
    "version": "1.0.0",
    "authors": [
        ""
    ],
    "license": "",

    "paths": {
        "original": "original",
        "changes": "changes",
        "output": "changed"
    }
}
```
You may have noticed that there are 3 fields: original, changes and output. 
- The original field represents the file path to the original Json file that you cannot change.
- The changes path represents the changes folder which contains the Json patch files that are not intended to be written manually
- And the output path represents the place where you change the Json files

## Modifying the file(s)
When you want to do some modifications, run `json-revisor build`.

This will get the original Json file(s) from `original` and patches from `changes`. It will then apply those patches and put the output file into the `output` folder.

## Saving the changes
When you are finished modifying the Json file(s) in the `output` folder, run `json-revisor update` to generate the patches from the changes that you made and save them to the `changes` folder. You can then discard the `changed` folder and/or `.gitignore` it if you are using Git.
