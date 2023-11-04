# sgrep

AI tool for converting natural language pattern descriptions into executable regex patterns

## Usage

You will need to create an account at [OpenAI](https://openai.com/) and get an API key.

Then you can authenticate with sgrep using the following command:

```bash
sgrep --auth <api_key>
```

Then you can use sgrep to search for patterns in text.


```bash
cat file.txt | sgrep "all the lines that start with '-' and have the word 'hello'"
```