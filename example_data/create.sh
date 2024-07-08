#!/bin/bash

# 1. Web application configuration
cat << EOF > config_web_app.yaml
app_name: MyWebApp
version: 1.0.0
debug_mode: false
database:
  host: localhost
  port: 5432
  name: myapp_db
  user: admin
EOF

# 2. User profile
cat << EOF > user_profile.yaml
user:
  id: 12345
  name: John Doe
  email: john.doe@example.com
  preferences:
    theme: dark
    notifications: true
  roles:
    - user
    - editor
EOF

# 3. Blog post structure
cat << EOF > blog_post.yaml
post:
  title: "YAML: A Beginner's Guide"
  author: Jane Smith
  date: 2024-06-20
  tags:
    - programming
    - data formats
  content: |
    YAML is a human-readable data serialization format...
  comments:
    - user: user123
      text: Great article!
    - user: user456
      text: Very helpful, thanks!
EOF

# 4. CI/CD pipeline configuration
cat << EOF > ci_cd_pipeline.yaml
pipeline:
  stages:
    - build
    - test
    - deploy
  build:
    script: 
      - npm install
      - npm run build
  test:
    script:
      - npm run test
  deploy:
    script:
      - docker build -t myapp .
      - docker push myapp:latest
EOF

# 5. Server environment configuration
cat << EOF > server_environment.yaml
environment:
  name: production
  servers:
    - name: web-01
      ip: 192.168.1.10
      role: web
    - name: db-01
      ip: 192.168.1.11
      role: database
  firewall:
    allowed_ports:
      - 80
      - 443
      - 22
EOF

# 6. Product inventory
cat << EOF > product_inventory.yaml
inventory:
  - product_id: PRD001
    name: Laptop
    category: Electronics
    price: 999.99
    in_stock: 50
  - product_id: PRD002
    name: Desk Chair
    category: Furniture
    price: 199.99
    in_stock: 30
EOF

# 7. Restaurant menu
cat << EOF > restaurant_menu.yaml
menu:
  appetizers:
    - name: Garlic Bread
      price: 5.99
    - name: Mozzarella Sticks
      price: 7.99
  main_courses:
    - name: Spaghetti Bolognese
      price: 12.99
    - name: Grilled Salmon
      price: 15.99
  desserts:
    - name: Tiramisu
      price: 6.99
    - name: Chocolate Cake
      price: 5.99
EOF

# 8. Book metadata
cat << EOF > book_metadata.yaml
book:
  title: "The YAML Chronicles"
  author: "Alex Writer"
  isbn: "978-1234567890"
  publication_date: 2024-01-15
  genre:
    - Science Fiction
    - Technology
  chapters:
    - "The Beginning of Data"
    - "YAML vs. The World"
    - "Return of the Serialization"
EOF

# 9. Task list
cat << EOF > task_list.yaml
tasks:
  - name: Buy groceries
    priority: high
    due_date: 2024-06-22
  - name: Call dentist
    priority: medium
    due_date: 2024-06-25
  - name: Clean garage
    priority: low
    due_date: 2024-06-30
EOF

# 10. Weather forecast
cat << EOF > weather_forecast.yaml
weather_forecast:
  city: New York
  country: USA
  days:
    - date: 2024-06-21
      condition: Sunny
      high_temp: 28
      low_temp: 20
      humidity: 45
    - date: 2024-06-22
      condition: Partly Cloudy
      high_temp: 26
      low_temp: 19
      humidity: 50
EOF

echo "All YAML files have been created successfully."
