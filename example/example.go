package main

import (
	"fmt"

	"github.com/Mxmilu666/fuckmiit"
)

func main() {
	domain := "baidu.com"
	result, err := fuckmiit.QueryDomainDetail(domain, true)
	if err != nil {
		fmt.Printf("Error querying domain: %v\n", err)
		return
	}
	fmt.Printf("Query result for %s: %+v\n", domain, result.MainLicence)
}
