use std::collections::{BTreeMap, VecDeque};

use crate::order::*;

#[derive(Debug)]
pub enum BookError {
    OrderNotFound
}

type PriceLabel = OrderPrice;
type OrderQueue<'a> = VecDeque<&'a mut Order<'a>>;
pub type Side<'a> = BTreeMap<PriceLabel, OrderQueue<'a>>;

pub type BookId = u128;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Book<'a> {
    id: BookId,
    name: String,
    ticker: String,
    bids: Side<'a>,
    asks: Side<'a>,
    ltp: OrderPrice,
    has_traded: bool,
    order_ids: Vec<OrderId>
}

impl<'a> Book<'a> {
    pub fn new(id: BookId, name: String, ticker: String) -> Self {
        Book {
            id,
            name,
            ticker,
            bids: Side::new(),
            asks: Side::new(),
            ltp: 0,
            has_traded: false,
            order_ids: vec![]
        }
    }

    pub fn id(&self) -> BookId {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn ticker(&self) -> String {
        self.ticker.clone()
    }

    pub fn ltp(&self) -> Option<OrderPrice> {
        if self.has_traded {
            Some(self.ltp)
        } else {
            None
        }
    }

    pub fn submit(&mut self, order: &'a mut Order<'a>) ->
        Result<(), BookError> {
        let order_id: OrderId = order.id();
        let price_key: OrderPrice = order.price();
        let order_quantity: OrderQuantity = order.quantity();

        match order.r#type() {
            OrderType::Bid => {
                let mut matched: bool = false;
                
                for curr_queue in self.asks.values_mut() {
                    /*let mut curr_queue: &mut VecDeque<&'a mut Order> =
                        self.asks.get_mut(curr_price).unwrap();*/
                    
                    for _i in 0..curr_queue.len() {
                        let counter_order = curr_queue.pop_front().unwrap();
                        let mut counter_order_done: bool = false;
                        let curr_price: OrderPrice = counter_order.price();
                        
                        if curr_price <= price_key {
                            let counter_quantity: OrderPrice =
                                                    counter_order.quantity();
                            
                            if counter_quantity < order_quantity {
                                counter_order.owner().add_balance(curr_price *
                                                            counter_quantity);
                                counter_order.owner().take_holding(
                                                            self.ticker.clone(),
                                                            counter_quantity);

                                order.owner().take_balance(curr_price *
                                                            counter_quantity);
                                order.owner().add_holding(self.ticker.clone(),
                                                            counter_quantity);

                                /* remove counter order as it is consumed */
                                counter_order_done = true;
                            } else if counter_quantity == order_quantity {
                                counter_order.owner().add_balance(curr_price *
                                                            counter_quantity);
                                counter_order.owner().take_holding(
                                                            self.ticker.clone(),
                                                            counter_quantity);

                                order.owner().take_balance(curr_price *
                                                            counter_quantity);
                                order.owner().add_holding(self.ticker.clone(),
                                                            counter_quantity);
                                
                                /* remove counter order as it is consumed */
                                counter_order_done = true;
 
                                matched = true;
                                break;
                            } else {
                                counter_order.owner().take_balance(curr_price *
                                                            order_quantity);
                                counter_order.owner().add_holding(
                                                            self.ticker.clone(),
                                                            order_quantity);

                                order.owner().add_balance(curr_price *
                                                            order_quantity);
                                order.owner().take_holding(self.ticker.clone(),
                                                            order_quantity);
                                
                                matched = true;
                                break;
                            }
                       
                            /* restore counter order if not consumed */
                            if !counter_order_done {     
                                curr_queue.push_back(counter_order);
                            }
                        } else {
                            curr_queue.push_back(counter_order);
                        }
                    }
                }

                if !matched {
                    self.bids.insert(price_key, VecDeque::new());
                    self.bids.get_mut(&price_key).unwrap().push_back(order);
                    self.order_ids.push(order_id);
                } else {
                    self.has_traded = true;
                    self.ltp = price_key;
                }
            },
            OrderType::Ask => {
                let mut matched: bool = false;
                
                for curr_queue in self.bids.values_mut() {
                    /*let mut curr_queue: &mut VecDeque<&'a mut Order> =
                        self.asks.get_mut(curr_price).unwrap();*/
                    
                    for _i in 0..curr_queue.len() {
                        let counter_order = curr_queue.pop_front().unwrap();
                        let mut counter_order_done: bool = false;
                        let curr_price: OrderPrice = counter_order.price();
                        
                        if curr_price <= price_key {
                            let counter_quantity: OrderPrice =
                                                    counter_order.quantity();
                            
                            if counter_quantity < order_quantity {
                                counter_order.owner().take_balance(curr_price *
                                                            counter_quantity);
                                counter_order.owner().add_holding(
                                                            self.ticker.clone(),
                                                            counter_quantity);

                                order.owner().add_balance(curr_price *
                                                            counter_quantity);
                                order.owner().take_holding(self.ticker.clone(),
                                                            counter_quantity);

                                /* remove counter order as it is consumed */
                                counter_order_done = true;
                            } else if counter_quantity == order_quantity {
                                counter_order.owner().take_balance(curr_price *
                                                            counter_quantity);
                                counter_order.owner().add_holding(
                                                            self.ticker.clone(),
                                                            counter_quantity);

                                order.owner().add_balance(curr_price *
                                                            counter_quantity);
                                order.owner().take_holding(self.ticker.clone(),
                                                            counter_quantity);
                                
                                /* remove counter order as it is consumed */
                                counter_order_done = true;
 
                                matched = true;
                                break;
                            } else {
                                counter_order.owner().add_balance(curr_price *
                                                            order_quantity);
                                counter_order.owner().take_holding(
                                                            self.ticker.clone(),
                                                            order_quantity);

                                order.owner().take_balance(curr_price *
                                                            order_quantity);
                                order.owner().add_holding(self.ticker.clone(),
                                                            order_quantity);
                                
                                matched = true;
                                break;
                            }
                       
                            /* restore counter order if not consumed */
                            if !counter_order_done {     
                                curr_queue.push_back(counter_order);
                            }
                        } else {
                            curr_queue.push_back(counter_order);
                        }
                    }
                }

                if !matched {
                    self.asks.insert(price_key, VecDeque::new());
                    self.asks.get_mut(&price_key).unwrap().push_back(order);
                    self.order_ids.push(order_id);
                } else {
                    self.has_traded = true;
                    self.ltp = price_key;
                }
            }
        };

        Ok(())
    }

    pub fn cancel(&mut self, id: OrderId) -> Result<(), BookError> {
        if !self.order_ids.contains(&id) {
            return Err(BookError::OrderNotFound);
        }

        let mut index: usize = 0;

        for (curr_price, curr_queue) in self.bids.iter_mut() {
            for curr_order in curr_queue.iter() {
                if curr_order.id() == id {
                    break;
                }

                index += 1;
            }

            curr_queue.remove(index);
            return Ok(());
        }
        
        Ok(())
    } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::account::{Account, AccountHolding};

    #[test]
    fn test_submit_equal_orders() -> Result<(), BookError> {
        let mut holdings: HashMap<String, AccountHolding> = HashMap::new();
        holdings.insert("MSFT".to_string(), 20);
        
        let mut actual_account1: Account =
                Account::new(1, "John Doe".to_string(), 2500, HashMap::new());
        let mut actual_account2: Account =
                Account::new(2, "Jane Doe".to_string(), 0, holdings);
        let mut actual_order1: Order = 
                Order::new(1000, &mut actual_account1, OrderType::Bid, 125, 20);
        let mut actual_order2: Order =
                Order::new(1001, &mut actual_account2, OrderType::Ask, 125, 20);
        
        let mut actual_book: Book = Book::new(1,
            "Vereenigde Oostindische Compagnie".to_string(), "VOC".to_string());
        
        actual_book.submit(&mut actual_order1)?;
        actual_book.submit(&mut actual_order2)?;
        
        /*let mut expected_book: Book = Book::new(1,
            "Vereenigde Oostindische Compagnie".to_string(), "VOC".to_string());*/
        
        let expected_book: Book = Book {
            id: 1,
            name: "Vereenigde Oostindische Compagnie".to_string(),
            ticker: "VOC".to_string(),
            bids: Side::new(),
            asks: Side::new(),
            ltp: 125,
            has_traded: true,
            order_ids: vec![]
        };

        assert_eq!(actual_book, expected_book);
        assert_eq!(actual_account1.balance(), 0);
        assert_eq!(actual_account1.holding("MSFT".to_string()).unwrap(), 20);
        assert_eq!(actual_account2.balance(), 2500);
        assert_eq!(actual_account2.holding("MSFT".to_string()).unwrap(), 0);
        
        Ok(())
    }
}

