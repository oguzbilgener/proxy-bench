package main

import (
	"flag"
	"fmt"
	"io"
	"net"
)

func main() {

	listen := flag.String("listen", "127.0.0.1:20000", "The address to listen on")
	upstream := flag.String("upstream", "127.0.0.1:20002", "The address to connect to")
	flag.Parse()

	listener, err := net.Listen("tcp", *listen)
	if err != nil {
		panic(err)
	}

	for {
		client, err := listener.Accept()
		if err != nil {
			panic(err)
		}

		go func() {
			conn, err := net.Dial("tcp", *upstream)
			if err != nil {
				client.Close()
				fmt.Println("Failed to connect to upstream")
				fmt.Println(err)
				return
			}

			go func() {
				written, err := io.Copy(client, conn)
				if err != nil || written == 0 {
					fmt.Println(err)
					client.Close()
					conn.Close()
					return
				}
			}()

			go func() {
				written, err := io.Copy(conn, client)
				if err != nil || written == 0 {
					fmt.Println(err)
					client.Close()
					conn.Close()
					return
				}
			}()
		}()
	}

}
