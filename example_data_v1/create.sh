#!/bin/bash

# 1. Web application configuration
cat << EOF > config_web_app.yaml
app_name: CoolWebApp
version: 2.1.3
debug_mode: true
database:
  host: db.example.com
  port: 3306
  name: coolapp_prod
  user: dbmaster
EOF

# 2. User profile
cat << EOF > user_profile.yaml
user:
  id: 78901
  name: Alice Johnson
  email: alice.j@example.net
  preferences:
    theme: light
    notifications: false
  roles:
    - admin
    - developer
EOF

# 3. Blog post structure
cat << EOF > blog_post.yaml
post:
  title: "10 Must-Know Python Tricks"
  author: Bob Smith
  date: 2024-08-15
  tags:
    - python
    - coding tips
  content: |
    Python offers many powerful features that can make your code more efficient...
  comments:
    - user: coder42
      text: This saved me hours of work!
    - user: pythonista
      text: Great post, looking forward to more.
EOF

# 4. CI/CD pipeline configuration
cat << EOF > ci_cd_pipeline.yaml
pipeline:
  stages:
    - lint
    - build
    - test
    - deploy
  lint:
    script:
      - eslint .
  build:
    script: 
      - yarn install
      - yarn build
  test:
    script:
      - yarn test
  deploy:
    script:
      - docker-compose up -d
      - kubectl apply -f k8s-config.yaml
EOF

# 5. Server environment configuration
cat << EOF > server_environment.yaml
environment:
  name: staging
  servers:
    - name: app-server-01
      ip: 10.0.1.5
      role: application
    - name: db-server-01
      ip: 10.0.1.6
      role: database
  firewall:
    allowed_ports:
      - 8080
      - 5432
      - 22
EOF

# 6. Product inventory
cat << EOF > product_inventory.yaml
inventory:
  - product_id: TECH007
    name: Smartphone
    category: Electronics
    price: 799.99
    in_stock: 100
  - product_id: HOME101
    name: Coffee Maker
    category: Appliances
    price: 49.99
    in_stock: 75
EOF

# 7. Restaurant menu
cat << EOF > restaurant_menu.yaml
menu:
  starters:
    - name: Caesar Salad
      price: 8.99
    - name: Tomato Soup
      price: 6.99
  main_dishes:
    - name: Beef Burger
      price: 14.99
    - name: Vegetarian Pizza
      price: 13.99
  desserts:
    - name: Apple Pie
      price: 7.99
    - name: Ice Cream Sundae
      price: 6.99
EOF

# 8. Book metadata
cat << EOF > book_metadata.yaml
book:
  title: "The AI Revolution"
  author: "Dr. Sarah Tech"
  isbn: "978-0987654321"
  publication_date: 2024-09-01
  genre:
    - Non-fiction
    - Technology
  chapters:
    - "The Rise of Machine Learning"
    - "AI in Everyday Life"
    - "The Future of Work"
EOF

# 9. Task list
cat << EOF > task_list.yaml
tasks:
  - name: Prepare presentation
    priority: high
    due_date: 2024-07-05
  - name: Schedule team meeting
    priority: medium
    due_date: 2024-07-10
  - name: Review project proposal
    priority: high
    due_date: 2024-07-15
EOF

# 10. Weather forecast
cat << EOF > weather_forecast.yaml
weather_forecast:
  city: London
  country: UK
  days:
    - date: 2024-07-01
      condition: Rainy
      high_temp: 18
      low_temp: 12
      humidity: 80
    - date: 2024-07-02
      condition: Cloudy
      high_temp: 20
      low_temp: 14
      humidity: 75
EOF

echo "All modified YAML files have been created successfully."
