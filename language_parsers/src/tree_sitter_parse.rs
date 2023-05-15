// reference: https://github.com/Wilfred/difftastic/blob/84af470128adf82302d47749ab9dc33e0e6409b2/src/parse/tree_sitter_parser.rs

use std::collections::{HashSet, VecDeque};
use tree_sitter as ts;
use tree_sitter::TreeCursor;

extern "C" {
    fn tree_sitter_go() -> ts::Language;
    fn tree_sitter_hcl() -> ts::Language;
    fn tree_sitter_java() -> ts::Language;
    fn tree_sitter_python() -> ts::Language;
    fn tree_sitter_rust() -> ts::Language;
}

pub struct TreeSitterConfig {
    pub language: ts::Language,
}

pub enum Language {
    Go,
    Hcl,
    Java,
    Python,
    Rust,
}

// from enum Language to TreeSitterConfig
pub fn from_language(language: Language) -> TreeSitterConfig {
    match language {
        Language::Go => TreeSitterConfig {
            language: unsafe { tree_sitter_go() },
        },
        Language::Hcl => TreeSitterConfig {
            language: unsafe { tree_sitter_hcl() },
        },
        Language::Java => TreeSitterConfig {
            language: unsafe { tree_sitter_java() },
        },
        Language::Python => TreeSitterConfig {
            language: unsafe { tree_sitter_python() },
        },
        Language::Rust => TreeSitterConfig {
            language: unsafe { tree_sitter_rust() },
        },
    }
}

pub fn to_tree(src: &str, config: &TreeSitterConfig) -> Option<ts::Tree> {
    let mut parser = ts::Parser::new();
    parser
        .set_language(config.language)
        .expect("Incompatible tree-sitter version");
    parser.parse(src, None)
}

/// NodeWrapper wraps a ts::Node with parent and children pointers and convenience methods.
#[derive(Debug, Clone)]
struct NodeWrapper<'tree> {
    ts_node: ts::Node<'tree>,

    parent: Option<Box<NodeWrapper<'tree>>>,

    // child_index is the index of this node in its parent's children array
    child_index: Option<usize>,
}

impl<'tree> NodeWrapper<'tree> {
    fn new_root(ts_node: ts::Node<'tree>) -> NodeWrapper<'tree> {
        NodeWrapper {
            ts_node,
            parent: None,
            child_index: None,
        }
    }

    fn new(
        ts_node: ts::Node<'tree>,
        parent: Option<Box<NodeWrapper<'tree>>>,
        child_index: Option<usize>,
    ) -> NodeWrapper<'tree> {
        NodeWrapper {
            ts_node,
            parent,
            child_index,
        }
    }

    fn previous_sibling(&self) -> Option<NodeWrapper<'tree>> {
        if let Some(child_index) = self.child_index {
            if child_index > 0 {
                return Some(self.parent.clone().unwrap().child(child_index - 1));
            }
        }
        None
    }

    fn child(&self, index: usize) -> NodeWrapper<'tree> {
        NodeWrapper::new(
            self.ts_node.child(index).unwrap(),
            Some(Box::new(self.clone())),
            Some(index),
        )
    }

    fn children(&self, cursor: &mut TreeCursor<'tree>) -> Vec<NodeWrapper<'tree>> {
        self.ts_node
            .children(cursor)
            .enumerate()
            .map(|(i, ts_node)| NodeWrapper::new(ts_node, Some(Box::new(self.clone())), Some(i)))
            .collect()
    }

    fn kind(&self) -> &str {
        self.ts_node.kind()
    }

    fn start_position(&self) -> ts::Point {
        self.ts_node.start_position()
    }

    fn end_position(&self) -> ts::Point {
        self.ts_node.end_position()
    }

    fn utf8_text(&self, src: &str) -> String {
        self.ts_node.utf8_text(src.as_bytes()).unwrap().to_string()
    }
}

fn test_go_parse() {
    let source_code = r#"
/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

package test

import (
	"context"
	"fmt"
	"github.com/asimihsan/auth_service/service/internal/crypto"
	"github.com/asimihsan/auth_service/service/internal/repository"
	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/aws/aws-sdk-go-v2/service/dynamodb"
	"github.com/aws/aws-sdk-go-v2/service/dynamodb/types"
	"github.com/aws/aws-sdk-go-v2/service/kms"
	"github.com/aws/aws-sdk-go-v2/service/ses"
	"github.com/hashicorp/go-multierror"
	"github.com/pkg/errors"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
	"os"
	"sync"
	"testing"
)

type SetupConfig struct {
	usersTableName               repository.UsersTableName
	usernamesTableName           repository.UsernamesTableName
	emailsTableName              repository.EmailsTableName
	passwordResetTokensTableName repository.PasswordResetTokensTableName
	siteName                     string
}

type SetupFixture struct {
	config              SetupConfig
	localstackContainer testcontainers.Container
	dynamoDBClient      *dynamodb.Client
	sesClient           *ses.Client
	kmsClient           *kms.Client
	endpoint            string
	correctPasswordHash string
	userRepo            repository.UserRepository
	secretRepo          repository.SecretRepository
}

func Setup(t *testing.T, setupConfig *SetupConfig) (*SetupFixture, error) {
	region := "us-west-2"
	ctx := context.Background()

	// Set up Localstack localstackContainer with Pro API key
	apiKey, ok := os.LookupEnv("LOCALSTACK_API_KEY")
	if !ok {
		return nil, fmt.Errorf("LOCALSTACK_API_KEY not set")
	}

	req := testcontainers.ContainerRequest{
		Image:        "localstack/localstack-pro:2.0.2-arm64",
		ExposedPorts: []string{"4566/tcp"},
		Env: map[string]string{
			"SERVICES":           "dynamodb",
			"DEFAULT_REGION":     region,
			"LOCALSTACK_API_KEY": apiKey,
		},
		WaitingFor: wait.ForLog("Ready."),
	}

	localstackContainer, err := testcontainers.GenericContainer(
		ctx,
		testcontainers.GenericContainerRequest{
			ContainerRequest: req,
			Started:          true,
		},
	)
	if err != nil {
		return nil, errors.WithMessage(err, "failed to start localstack localstackContainer")
	}

	host, err := localstackContainer.Host(ctx)
	if err != nil {
		return nil, errors.WithMessage(err, "failed to get localstack host")
	}
	port, err := localstackContainer.MappedPort(ctx, "4566")
	if err != nil {
		return nil, errors.WithMessage(err, "failed to get localstack port")
	}
	endpoint := fmt.Sprintf("http://%s:%d", host, port.Int())

	customResolver := aws.EndpointResolverWithOptionsFunc(
		func(service, region string, opts ...interface{}) (aws.Endpoint, error) {
			return aws.Endpoint{
				PartitionID:   "aws",
				URL:           endpoint,
				SigningRegion: region,
			}, nil
		})

	awsCfg, err := config.LoadDefaultConfig(context.TODO(),
		config.WithRegion(region),
		config.WithEndpointResolverWithOptions(customResolver),
		config.WithCredentialsProvider(
			credentials.NewStaticCredentialsProvider("dummy", "dummy", "dummy")),
	)
	if err != nil {
		t.Fatalf("Failed to load AWS config: %v", err)
	}

	// Create AWS clients
	dynamoDBClient := dynamodb.NewFromConfig(awsCfg)
	sesClient := ses.NewFromConfig(awsCfg)
	kmsClient := kms.NewFromConfig(awsCfg)

	// Create the required tables
	err = createTables(ctx, dynamoDBClient, t, setupConfig)
	if err != nil {
		t.Fatalf("Failed to create tables: %v", err)
	}

	correctPasswordHash, err := crypto.GeneratePasswordHash("correct-password")
	if err != nil {
		t.Fatalf("Failed to generate password hash: %v", err)
	}

	userRepo := repository.NewDynamoDBUserRepository(
		dynamoDBClient,
		setupConfig.usersTableName,
		setupConfig.usernamesTableName,
		setupConfig.emailsTableName,
		setupConfig.passwordResetTokensTableName,
	)

	secretRepo := repository.NewKMSSecretRepository(kmsClient)

	return &SetupFixture{
		localstackContainer: localstackContainer,
		endpoint:            endpoint,
		dynamoDBClient:      dynamoDBClient,
		sesClient:           sesClient,
		kmsClient:           kmsClient,
		correctPasswordHash: correctPasswordHash,
		userRepo:            userRepo,
		secretRepo:          secretRepo,
	}, nil
}

func (s *SetupFixture) TearDown(t *testing.T) {
	ctx := context.TODO()
	err := s.localstackContainer.Terminate(ctx)
	if err != nil {
		t.Fatalf("Failed to terminate LocalStack container: %v", err)
	}

}

func createTable(
	ctx context.Context,
	client *dynamodb.Client,
	tableName string,
	keySchema []types.KeySchemaElement,
	attributeDefinitions []types.AttributeDefinition,
	localSecondaryIndexes *[]types.LocalSecondaryIndex,
) error {
	createTableInput := dynamodb.CreateTableInput{
		TableName:            aws.String(tableName),
		KeySchema:            keySchema,
		AttributeDefinitions: attributeDefinitions,
		ProvisionedThroughput: &types.ProvisionedThroughput{
			ReadCapacityUnits:  aws.Int64(5),
			WriteCapacityUnits: aws.Int64(5),
		},
	}
	if localSecondaryIndexes != nil {
		createTableInput.LocalSecondaryIndexes = *localSecondaryIndexes
	}

	_, err := client.CreateTable(ctx, &createTableInput)
	return err
}

func createTables(
	ctx context.Context,
	dynamoDBClient *dynamodb.Client,
	t *testing.T,
	setupConfig *SetupConfig,
) error {
	numberOfTables := 4

	// wait group to wait for all the tables to be created
	var wg sync.WaitGroup
	wg.Add(numberOfTables)

	// Create the required tables
	tableCreationErrs := make(chan error, numberOfTables)
	go func() {
		defer wg.Done()
		tableCreationErrs <- createTable(ctx, dynamoDBClient, string(setupConfig.usersTableName), []types.KeySchemaElement{
			{
				AttributeName: aws.String("userId"),
				KeyType:       types.KeyTypeHash,
			},
		}, []types.AttributeDefinition{
			{
				AttributeName: aws.String("userId"),
				AttributeType: types.ScalarAttributeTypeS,
			},
		},
			nil, // no local secondary indexes
		)
	}()
	go func() {
		defer wg.Done()
		tableCreationErrs <- createTable(ctx, dynamoDBClient, string(setupConfig.emailsTableName), []types.KeySchemaElement{
			{
				AttributeName: aws.String("email"),
				KeyType:       types.KeyTypeHash,
			},
		}, []types.AttributeDefinition{
			{
				AttributeName: aws.String("email"),
				AttributeType: types.ScalarAttributeTypeS,
			},
		},
			nil, // no local secondary indexes
		)
	}()
	go func() {
		defer wg.Done()
		tableCreationErrs <- createTable(ctx, dynamoDBClient, string(setupConfig.usernamesTableName), []types.KeySchemaElement{
			{
				AttributeName: aws.String("username"),
				KeyType:       types.KeyTypeHash,
			},
		}, []types.AttributeDefinition{
			{
				AttributeName: aws.String("username"),
				AttributeType: types.ScalarAttributeTypeS,
			},
		},
			nil, // no local secondary indexes
		)
	}()
	go func() {
		defer wg.Done()
		tableCreationErrs <- createTable(ctx, dynamoDBClient, string(setupConfig.passwordResetTokensTableName), []types.KeySchemaElement{
			{
				AttributeName: aws.String("email"),
				KeyType:       types.KeyTypeHash,
			},
			{
				AttributeName: aws.String("createdAt"),
				KeyType:       types.KeyTypeRange,
			},
		}, []types.AttributeDefinition{
			{
				AttributeName: aws.String("email"),
				AttributeType: types.ScalarAttributeTypeS,
			},
			{
				AttributeName: aws.String("createdAt"),
				AttributeType: types.ScalarAttributeTypeN,
			},
			{
				AttributeName: aws.String("token"),
				AttributeType: types.ScalarAttributeTypeS,
			},
		},
			// LSI mapping email to token so that we can query by token
			&[]types.LocalSecondaryIndex{
				{
					IndexName: aws.String("email-token-index"),
					KeySchema: []types.KeySchemaElement{
						{
							AttributeName: aws.String("email"),
							KeyType:       types.KeyTypeHash,
						},
						{
							AttributeName: aws.String("token"),
							KeyType:       types.KeyTypeRange,
						},
					},
					Projection: &types.Projection{
						ProjectionType: types.ProjectionTypeAll,
					},
				},
			},
		)
	}()

	// Wait for all the tables to be created
	wg.Wait()

	// Check if any of the table creations failed
	close(tableCreationErrs)

	var multiErr *multierror.Error
	for err := range tableCreationErrs {
		if err != nil {
			t.Errorf("Failed to create table: %v", err)
			multiErr = multierror.Append(multiErr, err)
		}
	}

	return multiErr.ErrorOrNil()
}
"#;
    let config = from_language(Language::Go);
    let tree = to_tree(source_code, &config).unwrap();
    let root_node = tree.root_node();

    // hashset of node kind strings to detect and traverse
    let mut node_kinds: HashSet<String> = HashSet::new();
    node_kinds.insert("source_file".to_string());
    node_kinds.insert("function_declaration".to_string());
    node_kinds.insert("type_declaration".to_string());
    node_kinds.insert("type_spec".to_string());
    node_kinds.insert("method_declaration".to_string());

    let cursor = &mut root_node.walk();
    let mut queue: VecDeque<NodeWrapper> = VecDeque::new();
    queue.push_back(NodeWrapper::new_root(root_node));
    loop {
        if queue.is_empty() {
            break;
        }
        let node = queue.pop_front().unwrap();
        println!(
            "{}: {}:{} to {}:{}",
            node.kind(),
            node.start_position().row,
            node.start_position().column,
            node.end_position().row,
            node.end_position().column,
        );
        let node_kind = node.kind();

        // when you see method_declaration, child are func, parameter_list, field_identifier parameter_list, block
        // you want all children, but for block replace it with {\n// ...\n}
        if node_kind == "method_declaration" {
            let mut content = String::new();
            for child in node.children(cursor) {
                if child.kind() == "block" {
                    content.push_str(" {\n\t// ...\n}");
                } else {
                    if child.kind() != "parameter_list" {
                        content.push(' ');
                    }
                    content.push_str(&child.utf8_text(source_code));
                }
            }
            println!("method content: {}", content);
        }

        if node_kinds.contains(node_kind) {
            println!("found node kind: {}", node_kind);
            for child in node.children(cursor) {
                println!("child kind: {}", child.kind());
                let child_source = child.utf8_text(source_code);
                if node.kind() == "type_declaration"
                    && child.kind() == "type"
                    && child_source == "struct"
                {
                    println!("found struct");
                }

                queue.push_front(child.clone());

                // when you see struct type, here is how to go back up and get the struct content
                if child.kind() == "struct_type" {
                    let parent = &child.parent.clone().unwrap();
                    let grandparent = &parent.parent.clone().unwrap();
                    let content = grandparent.utf8_text(source_code);
                    println!("struct content: {}", content);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tree_sitter_parse::test_go_parse;

    #[test]
    fn test_parse() {
        test_go_parse();
    }
}
