# Vocab Quizzer
A tool to help quiz vocab for any language.

## Adding a Dictionary
To add a dictionary you can import an xml file in the following format:
```xml
<?xml version="1.0" ?>

<dictionary>
  <title>Example Dictionary</title>
  <words>
    <word>
      <text>Apple</text>
      <pronunciation>/ˈapəl/</pronunciation>
      <definition>The round fruit which typically has thin red or green skin and crisp flesh.</definition>
    </word>
    <word>
      <text>Orange</text>
      <pronunciation>/ˈôrənj,ˈärənj/</pronunciation>
      <definition>A round juicy citrus fruit with a tough bright reddish-yellow rind.</definition>
    </word>
  </words>
<dictionary>
```
Note that it is assumed that the words are in order of commonality. The first word is the most common, the last word is the least common. The pronunciation tag is optional.
## Current Features
Not much tbh
## Planned Features
ill do this later...
