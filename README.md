# fuckmiit

## What's this

I don't know.

## Usage

```go
package main

import (
	"fmt"
	"log"

	"github.com/Mxmilu666/fuckmiit"
)

func main() {
	result, err := fuckmiit.QueryDomain("example.com")
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println(string(result))
}
```

or

```go
item, err := fuckmiit.QueryDomainDetail("example.com")
if err != nil {
	log.Fatal(err)
}

fmt.Println(item.ServiceLicence)
```

## Thank

- Daiyangcheng's dress img
- ChatGPT

## License
WTFPL License