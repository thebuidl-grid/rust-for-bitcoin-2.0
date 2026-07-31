* **Sanitize the grader scripts cleanly**
```bash
sed -i 's/\r$//' grader/*.sh
```
* **rewrite a completely clean .gitattributes to lock LF line endings permanently**
```bash
printf '*.sh text eol=lf\n' > .gitattributes

# verifies if the '*' above is added
cat .gitattributes
```

git commit --amend --no-edit