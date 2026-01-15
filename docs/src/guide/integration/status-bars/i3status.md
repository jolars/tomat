# i3status

For i3status compatible bars like i3bar, you can use a helper script. First, add this
to your i3status config:

```
order += "read_file tomat"

read_file tomat {
    path = "/tmp/tomat-status"
    format = "%content"
}
```

Helper script:

```bash
#!/bin/bash
while true; do
    tomat status --output plain > /tmp/tomat-status
    sleep 1
done
```
